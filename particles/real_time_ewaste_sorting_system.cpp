#include <opencv2/opencv.hpp>
#include <tensorflow/lite/interpreter.h>
#include <tensorflow/lite/model.h>
#include <iostream>
#include <vector>

// Global variables for model and camera
std::unique_ptr<tflite::Interpreter> interpreter;
cv::VideoCapture cap(0);

void loadModel(const std::string& modelPath) {
    std::unique_ptr<tflite::FlatBufferModel> model = tflite::FlatBufferModel::BuildFromFile(modelPath.c_str());
    tflite::ops::builtin::BuiltinOpResolver resolver;
    tflite::InterpreterBuilder builder(*model, resolver);
    builder(&interpreter);
    interpreter->AllocateTensors();
}

void processFrame(cv::Mat& frame) {
    cv::Mat resizedFrame;
    cv::resize(frame, resizedFrame, cv::Size(224, 224));
    cv::cvtColor(resizedFrame, resizedFrame, cv::COLOR_BGR2RGB);

    // Copy frame data to input tensor
    float* input = interpreter->typed_input_tensor<float>(0);
    for (int i = 0; i < 224 * 224 * 3; ++i) {
        input[i] = resizedFrame.data[i] / 255.0f;
    }

    // Run inference
    interpreter->Invoke();

    // Get output
    float* output = interpreter->typed_output_tensor<float>(0);
    int predictedClass = std::max_element(output, output + 10) - output;

    // Draw bounding box and label
    cv::rectangle(frame, cv::Rect(50, 50, 200, 200), cv::Scalar(0, 255, 0), 2);
    cv::putText(frame, std::to_string(predictedClass), cv::Point(50, 40), cv::FONT_HERSHEY_SIMPLEX, 1, cv::Scalar(0, 255, 0), 2);
}

int main() {
    loadModel("ewaste_sorting_model.tflite");

    while (true) {
        cv::Mat frame;
        cap >> frame;
        if (frame.empty()) break;

        processFrame(frame);
        cv::imshow("E-Waste Sorting", frame);

        if (cv::waitKey(1) == 27) break;
    }

    return 0;
}
